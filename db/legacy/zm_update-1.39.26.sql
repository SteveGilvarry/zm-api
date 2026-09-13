--
-- Consolidate Event_Summaries maintenance into the triggers on Events.
--
-- Events_Hour/Day/Week/Month each carried their own update and delete
-- trigger, and every one of them issued a separate UPDATE against the same
-- Event_Summaries row. A single Event delete therefore locked that row five
-- times, which deadlocked against concurrent writers -- notably zmstats.pl
-- bulk-deleting aged rows out of Events_Hour.
--
-- event_update_trigger and event_delete_trigger now modify the bucket tables
-- themselves and apply one consolidated UPDATE, using ROW_COUNT() to tell
-- whether the event was still in each bucket so aged-out events do not
-- over-adjust the counters.
--
-- Consequence worth knowing: a direct DELETE against a bucket table (which is
-- how zmstats.pl prunes) no longer adjusts Event_Summaries at all. zmstats.pl
-- resyncs the Hour/Day/Week/Month columns from COUNT(*)/SUM(DiskSpace) on the
-- same pass it prunes, so those columns are eventually consistent within one
-- ZM_STATS_UPDATE_INTERVAL rather than exact at every instant. The Total and
-- Archived columns stay exact, because only the Events triggers touch them.
--

SELECT 'Dropping the per-bucket cascade triggers';

DROP TRIGGER IF EXISTS Events_Hour_delete_trigger;
DROP TRIGGER IF EXISTS Events_Hour_update_trigger;
DROP TRIGGER IF EXISTS Events_Day_delete_trigger;
DROP TRIGGER IF EXISTS Events_Day_update_trigger;
DROP TRIGGER IF EXISTS Events_Week_delete_trigger;
DROP TRIGGER IF EXISTS Events_Week_update_trigger;
DROP TRIGGER IF EXISTS Events_Month_delete_trigger;
DROP TRIGGER IF EXISTS Events_Month_update_trigger;

SELECT 'Resyncing Event_Summaries from the events themselves';

-- One-time resync so the new triggers start from ground truth rather than
-- from whatever the old cascade had accumulated. Also repairs the
-- ArchivedEventDiskSpace drift the old event_update_trigger left behind: its
-- only reachable branch for an already-archived event changing size updated
-- Events_Archived but not Event_Summaries.
UPDATE Event_Summaries es
LEFT JOIN (SELECT MonitorId, COUNT(*) c, COALESCE(SUM(DiskSpace),0) s FROM Events_Hour     GROUP BY MonitorId) h ON h.MonitorId = es.MonitorId
LEFT JOIN (SELECT MonitorId, COUNT(*) c, COALESCE(SUM(DiskSpace),0) s FROM Events_Day      GROUP BY MonitorId) d ON d.MonitorId = es.MonitorId
LEFT JOIN (SELECT MonitorId, COUNT(*) c, COALESCE(SUM(DiskSpace),0) s FROM Events_Week     GROUP BY MonitorId) w ON w.MonitorId = es.MonitorId
LEFT JOIN (SELECT MonitorId, COUNT(*) c, COALESCE(SUM(DiskSpace),0) s FROM Events_Month    GROUP BY MonitorId) m ON m.MonitorId = es.MonitorId
LEFT JOIN (SELECT MonitorId, COUNT(*) c, COALESCE(SUM(DiskSpace),0) s FROM Events_Archived GROUP BY MonitorId) a ON a.MonitorId = es.MonitorId
LEFT JOIN (SELECT MonitorId, COUNT(*) c, COALESCE(SUM(DiskSpace),0) s FROM Events          GROUP BY MonitorId) t ON t.MonitorId = es.MonitorId
SET
  es.HourEvents             = COALESCE(h.c,0),
  es.HourEventDiskSpace     = COALESCE(h.s,0),
  es.DayEvents              = COALESCE(d.c,0),
  es.DayEventDiskSpace      = COALESCE(d.s,0),
  es.WeekEvents             = COALESCE(w.c,0),
  es.WeekEventDiskSpace     = COALESCE(w.s,0),
  es.MonthEvents            = COALESCE(m.c,0),
  es.MonthEventDiskSpace    = COALESCE(m.s,0),
  es.ArchivedEvents         = COALESCE(a.c,0),
  es.ArchivedEventDiskSpace = COALESCE(a.s,0),
  es.TotalEvents            = COALESCE(t.c,0),
  es.TotalEventDiskSpace    = COALESCE(t.s,0);

-- Installs the consolidated event_update_trigger and event_delete_trigger,
-- and drops the bucket triggers again for good measure.

delimiter //

/* ============================================================================
 * Trigger architecture overview
 *
 * Event_Summaries is maintained exclusively by the event_* triggers on Events.
 * The old cascade triggers on Events_Hour / Events_Day / Events_Week /
 * Events_Month have been removed: each one previously fired a separate
 * UPDATE against Event_Summaries, so a single Event DELETE caused 5 locks
 * on the same Event_Summaries row and deadlocked with concurrent writers
 * (notably zmstats.pl bulk-deleting aged rows from Events_Hour).
 *
 * The event_update_trigger / event_delete_trigger on Events now:
 *   1. Modify the bucket tables (Events_Hour/Day/Week/Month) directly
 *   2. Use ROW_COUNT() to detect whether the event was still in each bucket
 *      (events that have aged out won't get their counters over-adjusted)
 *   3. Apply one consolidated UPDATE to Event_Summaries
 *
 * Inserts into Events_Hour/Day/Week/Month are performed by zm_event.cpp
 * directly (event_insert_trigger remains disabled), so no insert cascade
 * exists.
 *
 * Direct DELETEs against Events_Hour/Day/Week/Month (e.g. zmstats.pl pruning
 * aged rows) no longer touch Event_Summaries. zmstats.pl resyncs the
 * Hour/Day/Week/Month counter and disk-space columns from COUNT(*)/SUM(DiskSpace)
 * after pruning, which also self-heals any drift.
 * ============================================================================ */

/* ============================================================================
 * EVENT UPDATE TRIGGER
 * - Propagates DiskSpace changes to Events_Hour/Day/Week/Month rows
 * - Uses ROW_COUNT() per bucket so aged-out events don't over-adjust counters
 * - Handles Archived transitions and Archived DiskSpace updates
 *
 * Lock-acquisition order (same for BEFORE / AFTER triggers since InnoDB
 * X-locks the matched Events row during WHERE evaluation, before the
 * trigger body fires):
 *   Events[Id] -> buckets[Id] -> Event_Summaries[MonitorId]
 * event_delete_trigger does the same. zmstats.pl follows the matching
 * prefix (bucket DELETEs then UPDATE Event_Summaries) and crucially does
 * NOT pre-lock Event_Summaries — pre-locking ES would invert against the
 * trigger body order and reintroduce deadlocks against filter/zma.
 * ============================================================================ */
DROP TRIGGER IF EXISTS event_update_trigger//

CREATE TRIGGER event_update_trigger AFTER UPDATE ON Events
FOR EACH ROW
BEGIN
  DECLARE diff BIGINT DEFAULT 0;
  DECLARE hour_rows  INT DEFAULT 0;
  DECLARE day_rows   INT DEFAULT 0;
  DECLARE week_rows  INT DEFAULT 0;
  DECLARE month_rows INT DEFAULT 0;

  SET diff = COALESCE(NEW.DiskSpace, 0) - COALESCE(OLD.DiskSpace, 0);

  IF diff != 0 THEN
    UPDATE Events_Hour  SET DiskSpace = NEW.DiskSpace WHERE EventId = NEW.Id;
    SET hour_rows = ROW_COUNT();
    UPDATE Events_Day   SET DiskSpace = NEW.DiskSpace WHERE EventId = NEW.Id;
    SET day_rows = ROW_COUNT();
    UPDATE Events_Week  SET DiskSpace = NEW.DiskSpace WHERE EventId = NEW.Id;
    SET week_rows = ROW_COUNT();
    UPDATE Events_Month SET DiskSpace = NEW.DiskSpace WHERE EventId = NEW.Id;
    SET month_rows = ROW_COUNT();

    UPDATE Event_Summaries SET
      HourEventDiskSpace  = GREATEST(COALESCE(HourEventDiskSpace, 0)  + hour_rows  * diff, 0),
      DayEventDiskSpace   = GREATEST(COALESCE(DayEventDiskSpace, 0)   + day_rows   * diff, 0),
      WeekEventDiskSpace  = GREATEST(COALESCE(WeekEventDiskSpace, 0)  + week_rows  * diff, 0),
      MonthEventDiskSpace = GREATEST(COALESCE(MonthEventDiskSpace, 0) + month_rows * diff, 0),
      TotalEventDiskSpace = GREATEST(COALESCE(TotalEventDiskSpace, 0) + diff, 0)
    WHERE MonitorId = OLD.MonitorId;
  END IF;

  IF NEW.Archived != OLD.Archived THEN
    IF NEW.Archived THEN
      INSERT INTO Events_Archived (EventId, MonitorId, DiskSpace)
        VALUES (NEW.Id, NEW.MonitorId, NEW.DiskSpace);
      INSERT INTO Event_Summaries (MonitorId, ArchivedEvents, ArchivedEventDiskSpace)
        VALUES (NEW.MonitorId, 1, NEW.DiskSpace)
        ON DUPLICATE KEY UPDATE
          ArchivedEvents = COALESCE(ArchivedEvents, 0) + 1,
          ArchivedEventDiskSpace = COALESCE(ArchivedEventDiskSpace, 0) + COALESCE(NEW.DiskSpace, 0);
    ELSEIF OLD.Archived THEN
      DELETE FROM Events_Archived WHERE EventId = OLD.Id;
      UPDATE Event_Summaries SET
        ArchivedEvents = GREATEST(COALESCE(ArchivedEvents, 0) - 1, 0),
        ArchivedEventDiskSpace = GREATEST(COALESCE(ArchivedEventDiskSpace, 0) - COALESCE(OLD.DiskSpace, 0), 0)
      WHERE MonitorId = OLD.MonitorId;
    END IF;
  ELSEIF NEW.Archived AND diff != 0 THEN
    UPDATE Events_Archived SET DiskSpace = NEW.DiskSpace WHERE EventId = NEW.Id;
    UPDATE Event_Summaries SET
      ArchivedEventDiskSpace = GREATEST(COALESCE(ArchivedEventDiskSpace, 0) + diff, 0)
    WHERE MonitorId = OLD.MonitorId;
  END IF;
END;
//

/* event_insert_trigger intentionally omitted: zm_event.cpp inserts
 * Events_Hour/Day/Week/Month rows and bumps Event_Summaries counts as
 * part of event creation.
 */
DROP TRIGGER IF EXISTS event_insert_trigger//

/* ============================================================================
 * EVENT DELETE TRIGGER
 * - Removes from Events_Hour/Day/Week/Month and uses ROW_COUNT() so that
 *   aged-out events don't over-decrement counters
 * - One consolidated UPDATE against Event_Summaries
 * ============================================================================ */
DROP TRIGGER IF EXISTS event_delete_trigger//

CREATE TRIGGER event_delete_trigger BEFORE DELETE ON Events
FOR EACH ROW
BEGIN
  DECLARE hour_rows  INT DEFAULT 0;
  DECLARE day_rows   INT DEFAULT 0;
  DECLARE week_rows  INT DEFAULT 0;
  DECLARE month_rows INT DEFAULT 0;

  DELETE FROM Events_Hour  WHERE EventId = OLD.Id;  SET hour_rows  = ROW_COUNT();
  DELETE FROM Events_Day   WHERE EventId = OLD.Id;  SET day_rows   = ROW_COUNT();
  DELETE FROM Events_Week  WHERE EventId = OLD.Id;  SET week_rows  = ROW_COUNT();
  DELETE FROM Events_Month WHERE EventId = OLD.Id;  SET month_rows = ROW_COUNT();

  IF OLD.Archived THEN
    DELETE FROM Events_Archived WHERE EventId = OLD.Id;
    UPDATE Event_Summaries SET
      HourEvents             = GREATEST(COALESCE(HourEvents, 0)             - hour_rows, 0),
      HourEventDiskSpace     = GREATEST(COALESCE(HourEventDiskSpace, 0)     - hour_rows  * COALESCE(OLD.DiskSpace, 0), 0),
      DayEvents              = GREATEST(COALESCE(DayEvents, 0)              - day_rows, 0),
      DayEventDiskSpace      = GREATEST(COALESCE(DayEventDiskSpace, 0)      - day_rows   * COALESCE(OLD.DiskSpace, 0), 0),
      WeekEvents             = GREATEST(COALESCE(WeekEvents, 0)             - week_rows, 0),
      WeekEventDiskSpace     = GREATEST(COALESCE(WeekEventDiskSpace, 0)     - week_rows  * COALESCE(OLD.DiskSpace, 0), 0),
      MonthEvents            = GREATEST(COALESCE(MonthEvents, 0)            - month_rows, 0),
      MonthEventDiskSpace    = GREATEST(COALESCE(MonthEventDiskSpace, 0)    - month_rows * COALESCE(OLD.DiskSpace, 0), 0),
      TotalEvents            = GREATEST(COALESCE(TotalEvents, 0) - 1, 0),
      TotalEventDiskSpace    = GREATEST(COALESCE(TotalEventDiskSpace, 0)    - COALESCE(OLD.DiskSpace, 0), 0),
      ArchivedEvents         = GREATEST(COALESCE(ArchivedEvents, 0) - 1, 0),
      ArchivedEventDiskSpace = GREATEST(COALESCE(ArchivedEventDiskSpace, 0) - COALESCE(OLD.DiskSpace, 0), 0)
    WHERE MonitorId = OLD.MonitorId;
  ELSE
    UPDATE Event_Summaries SET
      HourEvents          = GREATEST(COALESCE(HourEvents, 0)          - hour_rows, 0),
      HourEventDiskSpace  = GREATEST(COALESCE(HourEventDiskSpace, 0)  - hour_rows  * COALESCE(OLD.DiskSpace, 0), 0),
      DayEvents           = GREATEST(COALESCE(DayEvents, 0)           - day_rows, 0),
      DayEventDiskSpace   = GREATEST(COALESCE(DayEventDiskSpace, 0)   - day_rows   * COALESCE(OLD.DiskSpace, 0), 0),
      WeekEvents          = GREATEST(COALESCE(WeekEvents, 0)          - week_rows, 0),
      WeekEventDiskSpace  = GREATEST(COALESCE(WeekEventDiskSpace, 0)  - week_rows  * COALESCE(OLD.DiskSpace, 0), 0),
      MonthEvents         = GREATEST(COALESCE(MonthEvents, 0)         - month_rows, 0),
      MonthEventDiskSpace = GREATEST(COALESCE(MonthEventDiskSpace, 0) - month_rows * COALESCE(OLD.DiskSpace, 0), 0),
      TotalEvents         = GREATEST(COALESCE(TotalEvents, 0) - 1, 0),
      TotalEventDiskSpace = GREATEST(COALESCE(TotalEventDiskSpace, 0) - COALESCE(OLD.DiskSpace, 0), 0)
    WHERE MonitorId = OLD.MonitorId;
  END IF;
END;
//

/* Drop the old cascade triggers on Events_Hour/Day/Week/Month - their
 * work is now done by event_update_trigger / event_delete_trigger.
 */
DROP TRIGGER IF EXISTS Events_Hour_delete_trigger//
DROP TRIGGER IF EXISTS Events_Hour_update_trigger//
DROP TRIGGER IF EXISTS Events_Day_delete_trigger//
DROP TRIGGER IF EXISTS Events_Day_update_trigger//
DROP TRIGGER IF EXISTS Events_Week_delete_trigger//
DROP TRIGGER IF EXISTS Events_Week_update_trigger//
DROP TRIGGER IF EXISTS Events_Month_delete_trigger//
DROP TRIGGER IF EXISTS Events_Month_update_trigger//

DROP PROCEDURE IF EXISTS update_storage_stats//

/* ============================================================================
 * ZONE TRIGGERS
 * Maintain ZoneCount on Monitors.
 * ============================================================================ */
DROP TRIGGER IF EXISTS Zone_Insert_Trigger//
CREATE TRIGGER Zone_Insert_Trigger AFTER INSERT ON Zones
FOR EACH ROW
  BEGIN
    UPDATE Monitors SET ZoneCount=(SELECT COUNT(*) FROM Zones WHERE MonitorId=NEW.MonitorId) WHERE Monitors.Id=NEW.MonitorID;
  END
//
DROP TRIGGER IF EXISTS Zone_Delete_Trigger//
CREATE TRIGGER Zone_Delete_Trigger AFTER DELETE ON Zones
FOR EACH ROW
  BEGIN
    UPDATE Monitors SET ZoneCount=(SELECT COUNT(*) FROM Zones WHERE MonitorId=OLD.MonitorId) WHERE Monitors.Id=OLD.MonitorID;
  END
//

DELIMITER ;
