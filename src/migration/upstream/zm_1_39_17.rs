//! Mirrors `db/legacy/zm_update-1.39.17.sql`: the object-detection columns
//! on `Monitors`, the five `AI_*` tables, the COCO dataset and its default
//! detection settings.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, DatabaseBackend};

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const FRAMEWORKS: [&str; 6] = [
    "TensorFlow",
    "PyTorch",
    "ONNX",
    "OpenVINO",
    "TensorRT",
    "Other",
];

const COCO: [&str; 80] = [
    "person",
    "bicycle",
    "car",
    "motorcycle",
    "airplane",
    "bus",
    "train",
    "truck",
    "boat",
    "traffic light",
    "fire hydrant",
    "stop sign",
    "parking meter",
    "bench",
    "bird",
    "cat",
    "dog",
    "horse",
    "sheep",
    "cow",
    "elephant",
    "bear",
    "zebra",
    "giraffe",
    "backpack",
    "umbrella",
    "handbag",
    "tie",
    "suitcase",
    "frisbee",
    "skis",
    "snowboard",
    "sports ball",
    "kite",
    "baseball bat",
    "baseball glove",
    "skateboard",
    "surfboard",
    "tennis racket",
    "bottle",
    "wine glass",
    "cup",
    "fork",
    "knife",
    "spoon",
    "bowl",
    "banana",
    "apple",
    "sandwich",
    "orange",
    "broccoli",
    "carrot",
    "hot dog",
    "pizza",
    "donut",
    "cake",
    "chair",
    "couch",
    "potted plant",
    "bed",
    "dining table",
    "toilet",
    "tv",
    "laptop",
    "mouse",
    "remote",
    "keyboard",
    "cell phone",
    "microwave",
    "oven",
    "toaster",
    "sink",
    "refrigerator",
    "book",
    "clock",
    "vase",
    "scissors",
    "teddy bear",
    "hair drier",
    "toothbrush",
];

/// Upstream's `Description` is the class name with the first letter
/// capitalised — except `tv`, which is `TV`.
fn describe(class: &str) -> String {
    if class == "tv" {
        return "TV".into();
    }
    let mut chars = class.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn timestamp3(m: &SchemaManager<'_>, col: &str) -> ColumnDef {
    let mut d = ColumnDef::new(c(col));
    d.custom(Alias::new("timestamp(3)"));
    match backend(m) {
        DatabaseBackend::MySql => d.default(Expr::cust("CURRENT_TIMESTAMP(3)")),
        _ => d.default(Expr::current_timestamp()),
    };
    d
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column_if_missing(
            m,
            "Monitors",
            "AnalysisImageOpacity",
            ColumnDef::new(c("AnalysisImageOpacity"))
                .tiny_unsigned()
                .not_null()
                .default("128")
                .to_owned(),
        )
        .await?;
        add_column_if_missing(
            m,
            "Monitors",
            "ObjectDetection",
            ColumnDef::new(c("ObjectDetection"))
                .string_len(16)
                .not_null()
                .default("none")
                .to_owned(),
        )
        .await?;
        add_column_if_missing(
            m,
            "Monitors",
            "ObjectDetectionModel",
            ColumnDef::new(c("ObjectDetectionModel"))
                .string_len(255)
                .not_null()
                .default("")
                .to_owned(),
        )
        .await?;
        add_column_if_missing(
            m,
            "Monitors",
            "ObjectDetectionObjectThreshold",
            ColumnDef::new(c("ObjectDetectionObjectThreshold"))
                .float()
                .not_null()
                .default(0.4_f64)
                .to_owned(),
        )
        .await?;
        add_column_if_missing(
            m,
            "Monitors",
            "ObjectDetectionNMSThreshold",
            ColumnDef::new(c("ObjectDetectionNMSThreshold"))
                .float()
                .not_null()
                .default(0.25_f64)
                .to_owned(),
        )
        .await?;
        modify_column(
            m,
            "Monitors",
            ColumnDef::new(c("ObjectDetection"))
                .string_len(16)
                .not_null()
                .default("none")
                .to_owned(),
        )
        .await?;

        if !table_exists(m, "AI_Datasets").await? {
            create_table(
                m,
                Table::create()
                    .table(t("AI_Datasets"))
                    .col(autoinc_pk(m, "Id", false))
                    .col(ColumnDef::new(c("Name")).string_len(64).not_null())
                    .col(ColumnDef::new(c("Description")).text())
                    .col(ColumnDef::new(c("Version")).string_len(32))
                    .col(ColumnDef::new(c("NumClasses")).unsigned().not_null())
                    .to_owned(),
                "AI_Datasets",
                &[("AI_Datasets_Name_idx", &["Name"], true)],
            )
            .await?;
        }
        if !table_exists(m, "AI_Models").await? {
            ensure_enum_type(m, "ai_models_framework", &FRAMEWORKS).await?;
            create_table(
                m,
                Table::create()
                    .table(t("AI_Models"))
                    .col(autoinc_pk(m, "Id", false))
                    .col(ColumnDef::new(c("Name")).string_len(64).not_null())
                    .col(ColumnDef::new(c("Description")).text())
                    .col(ColumnDef::new(c("ModelPath")).string_len(255))
                    .col(
                        enum_col("Framework", "ai_models_framework", &FRAMEWORKS)
                            .not_null()
                            .default("ONNX"),
                    )
                    .col(ColumnDef::new(c("Version")).string_len(32))
                    .col(ColumnDef::new(c("DatasetId")).unsigned())
                    .col(
                        ColumnDef::new(c("Enabled"))
                            .tiny_unsigned()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(t("AI_Models"), c("DatasetId"))
                            .to(t("AI_Datasets"), c("Id"))
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
                "AI_Models",
                &[("AI_Models_Name_idx", &["Name"], true)],
            )
            .await?;
        }
        if !table_exists(m, "AI_Object_Classes").await? {
            create_table(
                m,
                Table::create()
                    .table(t("AI_Object_Classes"))
                    .col(autoinc_pk(m, "Id", false))
                    .col(ColumnDef::new(c("DatasetId")).unsigned().not_null())
                    .col(ColumnDef::new(c("ClassName")).string_len(64).not_null())
                    .col(ColumnDef::new(c("ClassIndex")).unsigned().not_null())
                    .col(ColumnDef::new(c("Description")).text())
                    .foreign_key(
                        ForeignKey::create()
                            .from(t("AI_Object_Classes"), c("DatasetId"))
                            .to(t("AI_Datasets"), c("Id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
                "AI_Object_Classes",
                &[
                    ("AI_Object_Classes_DatasetId_idx", &["DatasetId"], false),
                    (
                        "AI_Object_Classes_Dataset_Class_idx",
                        &["DatasetId", "ClassName"],
                        true,
                    ),
                ],
            )
            .await?;
        }
        if !table_exists(m, "AI_Detection_Settings").await? {
            create_table(
                m,
                Table::create()
                    .table(t("AI_Detection_Settings"))
                    .col(autoinc_pk(m, "Id", false))
                    .col(ColumnDef::new(c("MonitorId")).unsigned())
                    .col(ColumnDef::new(c("ObjectClassId")).unsigned().not_null())
                    .col(
                        ColumnDef::new(c("Enabled"))
                            .tiny_unsigned()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(c("ReportDetection"))
                            .tiny_unsigned()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(c("ConfidenceThreshold"))
                            .tiny_unsigned()
                            .not_null()
                            .default(50),
                    )
                    .col(
                        ColumnDef::new(c("BoxColor"))
                            .string_len(7)
                            .not_null()
                            .default("#FF0000"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(t("AI_Detection_Settings"), c("MonitorId"))
                            .to(t("Monitors"), c("Id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(t("AI_Detection_Settings"), c("ObjectClassId"))
                            .to(t("AI_Object_Classes"), c("Id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
                "AI_Detection_Settings",
                &[
                    ("AI_Detection_Settings_MonitorId_idx", &["MonitorId"], false),
                    (
                        "AI_Detection_Settings_ObjectClassId_idx",
                        &["ObjectClassId"],
                        false,
                    ),
                    (
                        "AI_Detection_Settings_Monitor_Object_idx",
                        &["MonitorId", "ObjectClassId"],
                        true,
                    ),
                ],
            )
            .await?;
        }
        if !table_exists(m, "AI_Detections").await? {
            create_table(
                m,
                Table::create()
                    .table(t("AI_Detections"))
                    .col(autoinc_pk(m, "Id", true))
                    .col(ColumnDef::new(c("EventId")).big_unsigned().not_null())
                    .col(ColumnDef::new(c("FrameId")).big_unsigned())
                    .col(ColumnDef::new(c("ObjectClassId")).unsigned().not_null())
                    .col(ColumnDef::new(c("Confidence")).decimal_len(5, 4).not_null())
                    .col(ColumnDef::new(c("BoundingBoxX")).unsigned())
                    .col(ColumnDef::new(c("BoundingBoxY")).unsigned())
                    .col(ColumnDef::new(c("BoundingBoxWidth")).unsigned())
                    .col(ColumnDef::new(c("BoundingBoxHeight")).unsigned())
                    .col(timestamp3(m, "DetectedAt"))
                    .foreign_key(
                        ForeignKey::create()
                            .from(t("AI_Detections"), c("EventId"))
                            .to(t("Events"), c("Id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(t("AI_Detections"), c("FrameId"))
                            .to(t("Frames"), c("Id"))
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(t("AI_Detections"), c("ObjectClassId"))
                            .to(t("AI_Object_Classes"), c("Id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
                "AI_Detections",
                &[
                    ("AI_Detections_EventId_idx", &["EventId"], false),
                    ("AI_Detections_FrameId_idx", &["FrameId"], false),
                    ("AI_Detections_ObjectClassId_idx", &["ObjectClassId"], false),
                ],
            )
            .await?;
        }

        // Seeds: the COCO dataset, its 80 classes, and default settings for
        // the five classes upstream enables out of the box.
        insert_if_absent(
            m,
            probe_eq("AI_Datasets", "Name", "COCO"),
            Query::insert()
                .into_table(t("AI_Datasets"))
                .columns([
                    c("Id"),
                    c("Name"),
                    c("Description"),
                    c("Version"),
                    c("NumClasses"),
                ])
                .values_panic([
                    1.into(),
                    "COCO".into(),
                    "Microsoft Common Objects in Context".into(),
                    "2017".into(),
                    80.into(),
                ])
                .to_owned(),
        )
        .await?;
        let conn = m.get_connection();
        let dataset_id: i64 = conn
            .query_one(
                backend(m).build(
                    Query::select()
                        .expr_as(as_i64(m, "Id"), Alias::new("id"))
                        .from(t("AI_Datasets"))
                        .and_where(Expr::col(c("Name")).eq("COCO")),
                ),
            )
            .await?
            .map(|r| r.try_get::<i64>("", "id"))
            .transpose()?
            .unwrap_or(1);
        for (index, class) in COCO.iter().enumerate() {
            insert_if_absent(
                m,
                Query::select()
                    .expr(Expr::value(1))
                    .from(t("AI_Object_Classes"))
                    .and_where(Expr::col(c("DatasetId")).eq(dataset_id))
                    .and_where(Expr::col(c("ClassName")).eq(*class))
                    .to_owned(),
                Query::insert()
                    .into_table(t("AI_Object_Classes"))
                    .columns([
                        c("DatasetId"),
                        c("ClassName"),
                        c("ClassIndex"),
                        c("Description"),
                    ])
                    .values_panic([
                        dataset_id.into(),
                        (*class).into(),
                        (index as i64).into(),
                        describe(class).into(),
                    ])
                    .to_owned(),
            )
            .await?;
        }
        let defaults: [(&str, i32, &str); 5] = [
            ("person", 60, "#FF0000"),
            ("car", 50, "#0000FF"),
            ("truck", 50, "#0066FF"),
            ("bus", 50, "#0099FF"),
            ("motorcycle", 50, "#00CCFF"),
        ];
        for (class, threshold, colour) in defaults {
            let class_id: Option<i64> = conn
                .query_one(
                    backend(m).build(
                        Query::select()
                            .expr_as(as_i64(m, "Id"), Alias::new("id"))
                            .from(t("AI_Object_Classes"))
                            .and_where(Expr::col(c("DatasetId")).eq(dataset_id))
                            .and_where(Expr::col(c("ClassName")).eq(class)),
                    ),
                )
                .await?
                .map(|r| r.try_get::<i64>("", "id"))
                .transpose()?;
            let Some(class_id) = class_id else { continue };
            insert_if_absent(
                m,
                Query::select()
                    .expr(Expr::value(1))
                    .from(t("AI_Detection_Settings"))
                    .and_where(Expr::col(c("MonitorId")).is_null())
                    .and_where(Expr::col(c("ObjectClassId")).eq(class_id))
                    .to_owned(),
                Query::insert()
                    .into_table(t("AI_Detection_Settings"))
                    .columns([
                        c("MonitorId"),
                        c("ObjectClassId"),
                        c("Enabled"),
                        c("ReportDetection"),
                        c("ConfidenceThreshold"),
                        c("BoxColor"),
                    ])
                    .values_panic([
                        Option::<i64>::None.into(),
                        class_id.into(),
                        1.into(),
                        1.into(),
                        threshold.into(),
                        colour.into(),
                    ])
                    .to_owned(),
            )
            .await?;
        }
        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn descriptions_capitalise_like_upstream() {
        assert_eq!(super::describe("traffic light"), "Traffic light");
        assert_eq!(super::describe("tv"), "TV");
    }
}
