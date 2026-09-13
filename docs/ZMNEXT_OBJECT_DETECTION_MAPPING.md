# ZoneMinder 1.39.17 object detection vs zm-next pipelines

Status: **Analysis, nothing wired.** Written 2026-09-13. On hold until it's
settled with the ZoneMinder maintainer whether zm-next is the engine behind this
schema.

## Question

ZoneMinder 1.39.17 (`db/zm_update-1.39.17.sql`) adds per-monitor object
detection settings and a model/class registry. zm-api already exposes them
(`Monitors` fields through `GET/PATCH /monitors`; `/api/v3/ai/{datasets,models,object-classes}`
CRUD). Do they map cleanly onto the pipeline JSON that
`src/service/zmnext/pipeline.rs` generates?

**Short answer: no.** Two fields map directly (model path, confidence
threshold). The backend name, NMS threshold, per-class settings and detections
table don't have a clean home, and wiring even the two direct fields changes
thresholds for every monitor unless it's gated.

## What each side has

**ZoneMinder** (checked at `ZoneMinder@5d92a58c`, 2026-09-13):

- `Monitors.ObjectDetection VARCHAR(16) NOT NULL DEFAULT 'none'`: free text by
  design ("backends can be added without a schema change"). No list of valid
  values exists.
- `Monitors.ObjectDetectionModel VARCHAR(255) NOT NULL DEFAULT ''`: model path
  or name.
- `Monitors.ObjectDetectionObjectThreshold FLOAT NOT NULL DEFAULT 0.4`.
- `Monitors.ObjectDetectionNMSThreshold FLOAT NOT NULL DEFAULT 0.25`.
- `AI_Datasets` (seeded with COCO 2017, 80 classes), `AI_Models` (`ModelPath`,
  `Framework` enum TensorFlow/PyTorch/ONNX/OpenVINO/TensorRT/Other, `DatasetId`,
  `Enabled`), `AI_Object_Classes` (`ClassName`, `ClassIndex` per dataset),
  `AI_Detection_Settings` (per monitor and class: `Enabled`, `ReportDetection`,
  `ConfidenceThreshold TINYINT` 0–100 default 50, `BoxColor`), `AI_Detections`
  (per-box rows linked to Events/Frames).
- Nothing in ZoneMinder's `src/` or `web/includes/` reads any of it yet. The PHP
  UI only has admin grids for the three registry tables. So the columns carry no
  established runtime meaning to stay compatible with.

**zm-api generator** (`pipeline.rs:210`): the `decode_detect` node's cfg comes
entirely from global `[zmnext.pipeline]` config: `model_path`
(`/var/lib/zm-api/models/yolo26n.onnx`), `hw` (`auto`), `input_size` (640),
`conf_threshold` (0.35), plus a hardcoded `roi_motion: true`. `daemon/manager.rs`
has the monitor row in hand at generation time but reads none of the
`ObjectDetection*` fields. A stored per-monitor graph (`monitor_pipeline` table)
can set any cfg by hand, and that's the only per-monitor override today.

**zm-next detect plugins:**

- `decode_detect` (`decode_detect.cpp:134`): `model_path`, `input_size` (640),
  `conf_threshold` (0.25), `hw` (`auto` → `metal` on macOS, else
  `cuda`/`vaapi`/`vulkan`/`openvino`), `class_filter` (numeric class ids),
  `roi_motion`. No class-names key (COCO-80 is compiled in). No NMS key: it only
  decodes the YOLO26 end-to-end `[1,N,6]` output.
- `detect_onnx` (`detect_onnx.cpp:139`): same model/size/threshold keys, `ep`
  (`cpu`/`coreml`/`cuda`), `class_filter`, `class_names` (defaults to COCO-80).
  Also YOLO26-only; a fixed, unconfigurable 0.5-IoU overlap merge runs after
  decode.
- `detect_pose` and `detect_seg` do have `iou_threshold` (default 0.45): their
  heads aren't end-to-end, so they run real NMS.

## Mapping

| ZoneMinder | zm-next cfg | Fit | Notes |
|---|---|---|---|
| `Monitors.ObjectDetection` = `'none'` | no detect node (or leave the stored graph alone) | Partial | Needs a decision: does `'none'` remove detection from a zm-next monitor, or mean "not configured, use zm-api defaults"? Every existing row is `'none'`, so the first reading would switch detection off everywhere. |
| `Monitors.ObjectDetection` = backend name | plugin `kind` plus `hw` / `ep` | **Doesn't fit** | zm-next splits this into a plugin choice (`decode_detect` for GPU zero-copy, `detect_onnx` for ONNX Runtime) and a backend within it, with different vocabularies (`hw`: metal/cuda/vaapi/vulkan/openvino; `ep`: cpu/coreml/cuda). VARCHAR(16) can hold e.g. `zmnext:cuda`, but that's an invented convention ZoneMinder doesn't know about. |
| `Monitors.ObjectDetectionModel` | `model_path` | **Clean** | Fills a field the generator takes from global config. Empty string (the default) → keep the global default. |
| `AI_Models.ModelPath` (via name lookup) | `model_path` | Partial | Nothing links a monitor to an `AI_Models` row: `ObjectDetectionModel` is a path or name, not a foreign key. Resolving "name → row → ModelPath" is a guess at intent. |
| `AI_Models.Framework` | plugin `kind` / `ep` | **Doesn't fit** | ONNX is the only framework either plugin loads. TensorFlow/PyTorch/TensorRT rows can't run; OpenVINO is a `decode_detect` `hw` value, not a model format. |
| input size | `input_size` | **Missing in ZM** | No column. Stays global config (or read from ONNX input shape in zm-next). |
| `Monitors.ObjectDetectionObjectThreshold` | `conf_threshold` | **Clean, with a catch** | Same meaning, both 0–1. But the column defaults to 0.4 on every row while zm-api defaults to 0.35 and zm-next to 0.25, so wiring it unconditionally changes every monitor's sensitivity. |
| `Monitors.ObjectDetectionNMSThreshold` | none (`decode_detect`, `detect_onnx`); `iou_threshold` (`detect_pose`, `detect_seg`) | **Unused** | YOLO26 is NMS-free, so the default pipeline has nowhere to put it. Only meaningful if a monitor's graph uses a pose/seg detector. |
| `AI_Object_Classes` (dataset class list) | `class_names` (`detect_onnx` only) | Partial | `decode_detect` has COCO-80 compiled in and can't take names. Only matters for non-COCO models. |
| `AI_Detection_Settings.Enabled` per class | `class_filter` (ids) | Partial | Maps via `ClassIndex`. But rows are per monitor *or* global (`MonitorId NULL`), and zm-api has no entity for this table yet (GH #47). |
| `AI_Detection_Settings.ConfidenceThreshold` per class | none | **Doesn't fit** | zm-next has one threshold per detector. Per-class thresholds would need a zm-next change or a filter step in zm-api ingest. Also 0–100 integer vs 0–1 float. |
| `AI_Detection_Settings.ReportDetection`, `BoxColor` | none | **Doesn't fit** | UI/reporting concerns; could live in ingest and overlay config, not the detect node. |
| `AI_Detections` | zm-next `detection` EVENT (label, confidence, bbox, class_id) | Output, not config | A possible ingest target: `service/zmnext/ingest.rs` could write per-box rows here instead of only aggregating labels onto Events. Separate decision. |

## Doesn't fit, in short

1. **Backend name.** One free-text column vs zm-next's plugin plus backend pair
   with two vocabularies.
2. **NMS threshold.** No destination for the default YOLO26 pipeline.
3. **Framework enum.** Only ONNX runs.
4. **Per-class thresholds, report flags, box colours.** No zm-next equivalent.
5. **Model linkage.** `ObjectDetectionModel` is a string, not a key into
   `AI_Models`.
6. **Defaults.** 0.4 (ZM column) vs 0.35 (zm-api) vs 0.25 (zm-next), and
   `'none'` on every existing row.

## Possible narrow change (not made; needs your OK)

The one piece that only fills hardcoded fields: in `generate_pipeline`, when a
zm-next monitor has `ObjectDetection` set to something other than `'none'`, take
`model_path` from `ObjectDetectionModel` (when non-empty) and `conf_threshold`
from `ObjectDetectionObjectThreshold`; otherwise keep today's global config.
Gating on `ObjectDetection != 'none'` means no existing monitor changes
behaviour. It would touch `pipeline.rs`, the `generate_pipeline` call in
`daemon/manager.rs`, and tests. It doesn't settle what the backend value means,
so it's only worth doing once the maintainer conversation lands.
