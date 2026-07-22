SELECT 'This update may make changes that require SUPER privileges. If you see an error message saying:

ERROR 1419 (HY000) at line 298: You do not have the SUPER privilege and binary logging is enabled (you *might* want to use the less safe log_bin_trust_function_creators variable)

You will have to either run this update as root manually using something like (on ubuntu/debian)

sudo mysql --defaults-file=/etc/mysql/debian.cnf zm < /usr/share/zoneminder/db/zm_update-1.35.24.sql

OR

sudo mysql --defaults-file=/etc/mysql/debian.cnf "set global log_bin_trust_function_creators=1;"
sudo zmupdate.pl

OR

Turn off binary logging in your mysql server by adding this to your mysql config.
[mysqld]
skip-log-bin

';

SET @s = (SELECT IF(
    (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS WHERE table_schema = DATABASE()
      AND table_name = 'Event_Summaries'
    ) > 0
    ,
    "SELECT 'Event_Summaries Already exists'",
    "
    CREATE TABLE `Event_Summaries` (
  `MonitorId` int(10) unsigned NOT NULL,
  `TotalEvents` int(10) default NULL,
  `TotalEventDiskSpace` bigint default NULL,
  `HourEvents` int(10) default NULL,
  `HourEventDiskSpace` bigint default NULL,
  `DayEvents` int(10) default NULL,
  `DayEventDiskSpace` bigint default NULL,
  `WeekEvents` int(10) default NULL,
  `WeekEventDiskSpace` bigint default NULL,
  `MonthEvents` int(10) default NULL,
  `MonthEventDiskSpace` bigint default NULL,
  `ArchivedEvents` int(10) default NULL,
  `ArchivedEventDiskSpace` bigint default NULL,
  PRIMARY KEY (`MonitorId`)
) ENGINE=InnoDb;
"
    ));

PREPARE stmt FROM @s;
EXECUTE stmt;

DELETE FROM Event_Summaries;

REPLACE INTO Event_Summaries
  SELECT  MonitorId,
    COUNT(Id) AS TotalEvents,
    SUM(DiskSpace) AS TotalEventDiskSpace,
    SUM(IF(StartDateTime > DATE_SUB(NOW(), INTERVAL 1 hour),1,0)) AS HourEvents,
    SUM(IF(StartDateTime > DATE_SUB(NOW(), INTERVAL 1 hour),DiskSpace,0)) AS HourEventDiskSpace,
    SUM(IF(StartDateTime > DATE_SUB(NOW(), INTERVAL 1 day),1,0)) AS DayEvents,
    SUM(IF(StartDateTime > DATE_SUB(NOW(), INTERVAL 1 day),DiskSpace,0)) AS DayEventDiskSpace,
    SUM(IF(StartDateTime > DATE_SUB(NOW(), INTERVAL 1 week),1,0)) AS WeekEvents,
    SUM(IF(StartDateTime > DATE_SUB(NOW(), INTERVAL 1 week),DiskSpace,0)) AS WeekEventDiskSpace,
    SUM(IF(StartDateTime > DATE_SUB(NOW(), INTERVAL 1 month),1,0)) AS MonthEvents,
    SUM(IF(StartDateTime > DATE_SUB(NOW(), INTERVAL 1 month),DiskSpace,0)) AS MonthEventDiskSpace,
    SUM(IF(Archived,1,0)) AS ArchivedEvents,
    SUM(IF(Archived,DiskSpace,0)) AS ArchivedEventDiskSpace
    FROM Events GROUP BY MonitorId;
   

delimiter //
DROP TRIGGER IF EXISTS Events_Hour_delete_trigger//
CREATE TRIGGER Events_Hour_delete_trigger BEFORE DELETE ON Events_Hour
FOR EACH ROW BEGIN
  UPDATE Event_Summaries SET
  HourEvents = GREATEST(COALESCE(HourEvents,1)-1,0),
  HourEventDiskSpace=GREATEST(COALESCE(HourEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0)
  WHERE Event_Summaries.MonitorId=OLD.MonitorId;
END; 
//

DROP TRIGGER IF EXISTS Events_Hour_update_trigger//

CREATE TRIGGER Events_Hour_update_trigger AFTER UPDATE ON Events_Hour
FOR EACH ROW
  BEGIN
    declare diff BIGINT default 0;

    set diff = COALESCE(NEW.DiskSpace,0) - COALESCE(OLD.DiskSpace,0);
    IF ( diff ) THEN
      IF ( NEW.MonitorID != OLD.MonitorID ) THEN
        UPDATE Event_Summaries SET HourEventDiskSpace=GREATEST(COALESCE(HourEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0) WHERE Event_Summaries.MonitorId=OLD.MonitorId;
        UPDATE Event_Summaries SET HourEventDiskSpace=COALESCE(HourEventDiskSpace,0)+COALESCE(NEW.DiskSpace,0) WHERE Event_Summaries.MonitorId=NEW.MonitorId;
      ELSE
        UPDATE Event_Summaries SET HourEventDiskSpace=COALESCE(HourEventDiskSpace,0)+diff WHERE Event_Summaries.MonitorId=NEW.MonitorId;
      END IF;
    END IF;
  END;
//

DROP TRIGGER IF EXISTS Events_Day_delete_trigger//
CREATE TRIGGER Events_Day_delete_trigger BEFORE DELETE ON Events_Day
FOR EACH ROW BEGIN
  UPDATE Event_Summaries SET
  DayEvents = GREATEST(COALESCE(DayEvents,1)-1,0),
  DayEventDiskSpace=GREATEST(COALESCE(DayEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0)
  WHERE Event_Summaries.MonitorId=OLD.MonitorId;
END;
//

DROP TRIGGER IF EXISTS Events_Day_update_trigger;
CREATE TRIGGER Events_Day_update_trigger AFTER UPDATE ON Events_Day
FOR EACH ROW
  BEGIN
    declare diff BIGINT default 0;

    set diff = COALESCE(NEW.DiskSpace,0) - COALESCE(OLD.DiskSpace,0);
    IF ( diff ) THEN
      IF ( NEW.MonitorID != OLD.MonitorID ) THEN
        UPDATE Event_Summaries SET DayEventDiskSpace=GREATEST(COALESCE(DayEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0) WHERE Event_Summaries.MonitorId=OLD.MonitorId;
        UPDATE Event_Summaries SET DayEventDiskSpace=COALESCE(DayEventDiskSpace,0)+COALESCE(NEW.DiskSpace,0) WHERE Event_Summaries.MonitorId=NEW.MonitorId;
      ELSE
        UPDATE Event_Summaries SET DayEventDiskSpace=GREATEST(COALESCE(DayEventDiskSpace,0)+diff,0) WHERE Event_Summaries.MonitorId=NEW.MonitorId;
      END IF;
    END IF;
  END;
  //


DROP TRIGGER IF EXISTS Events_Week_delete_trigger//
CREATE TRIGGER Events_Week_delete_trigger BEFORE DELETE ON Events_Week
FOR EACH ROW BEGIN
  UPDATE Event_Summaries SET
  WeekEvents = GREATEST(COALESCE(WeekEvents,1)-1,0),
  WeekEventDiskSpace=GREATEST(COALESCE(WeekEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0)
  WHERE Event_Summaries.MonitorId=OLD.MonitorId;
END;
//

DROP TRIGGER IF EXISTS Events_Week_update_trigger;
CREATE TRIGGER Events_Week_update_trigger AFTER UPDATE ON Events_Week
FOR EACH ROW
  BEGIN
    declare diff BIGINT default 0;

    set diff = COALESCE(NEW.DiskSpace,0) - COALESCE(OLD.DiskSpace,0);
    IF ( diff ) THEN
      IF ( NEW.MonitorID != OLD.MonitorID ) THEN
        UPDATE Event_Summaries SET WeekEventDiskSpace=GREATEST(COALESCE(WeekEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0) WHERE Event_Summaries.MonitorId=OLD.MonitorId;
        UPDATE Event_Summaries SET WeekEventDiskSpace=COALESCE(WeekEventDiskSpace,0)+COALESCE(NEW.DiskSpace,0) WHERE Event_Summaries.MonitorId=NEW.MonitorId;
      ELSE
        UPDATE Event_Summaries SET WeekEventDiskSpace=GREATEST(COALESCE(WeekEventDiskSpace,0)+diff,0) WHERE Event_Summaries.MonitorId=NEW.MonitorId;
      END IF;
    END IF;
  END;
  //

DROP TRIGGER IF EXISTS Events_Month_delete_trigger//
CREATE TRIGGER Events_Month_delete_trigger BEFORE DELETE ON Events_Month
FOR EACH ROW BEGIN
  UPDATE Event_Summaries SET
  MonthEvents = GREATEST(COALESCE(MonthEvents,1)-1,0),
  MonthEventDiskSpace=GREATEST(COALESCE(MonthEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0)
  WHERE Event_Summaries.MonitorId=OLD.MonitorId;
END;
//

DROP TRIGGER IF EXISTS Events_Month_update_trigger;
CREATE TRIGGER Events_Month_update_trigger AFTER UPDATE ON Events_Month
FOR EACH ROW
  BEGIN
    declare diff BIGINT default 0;

    set diff = COALESCE(NEW.DiskSpace,0) - COALESCE(OLD.DiskSpace,0);
    IF ( diff ) THEN
      IF ( NEW.MonitorID != OLD.MonitorID ) THEN
        UPDATE Event_Summaries SET MonthEventDiskSpace=GREATEST(COALESCE(MonthEventDiskSpace,0)-COALESCE(OLD.DiskSpace),0) WHERE Event_Summaries.MonitorId=OLD.MonitorId;
        UPDATE Event_Summaries SET MonthEventDiskSpace=COALESCE(MonthEventDiskSpace,0)+COALESCE(NEW.DiskSpace) WHERE Event_Summaries.MonitorId=NEW.MonitorId;
      ELSE
        UPDATE Event_Summaries SET MonthEventDiskSpace=GREATEST(COALESCE(MonthEventDiskSpace,0)+diff,0) WHERE Event_Summaries.MonitorId=NEW.MonitorId;
      END IF;
    END IF;
  END;
  //

drop procedure if exists update_storage_stats//

/* ============================================================================
 * Canonical lock-acquisition order for every writer that touches Events,
 * the bucket tables, and Event_Summaries. InnoDB X-locks the matched Events
 * row during WHERE evaluation, before either BEFORE or AFTER trigger bodies
 * fire, so the order is the same regardless of trigger timing:
 *
 *   Events[Id] -> Events_Hour/Day/Week/Month[EventId] -> Event_Summaries[MonitorId]
 *
 * Writers that follow this order (and must continue to):
 *   - event_update_trigger (AFTER UPDATE on Events)
 *   - event_delete_trigger (BEFORE DELETE on Events)
 *   - The Event::Event constructor in src/zm_event.cpp: INSERT Events, then
 *     INSERT Events_Hour/Day/Week/Month, then INSERT/UPDATE Event_Summaries
 *     (event_insert_trigger is commented out below; zmc does it directly)
 *   - The bucket update/delete triggers cascade into Event_Summaries in the
 *     same direction
 *   - zmstats.pl prune+resync (bucket DELETEs then UPDATE Event_Summaries)
 *   - zmaudit.pl resync (bucket SELECTs then UPDATE Event_Summaries)
 *
 * Crucially: do NOT pre-lock Event_Summaries before touching the bucket
 * tables — that inverts the order and reintroduces the deadlock cycle
 * against zma/filter/zmc writers.
 * ============================================================================ */
drop trigger if exists event_update_trigger//

CREATE TRIGGER event_update_trigger AFTER UPDATE ON Events
FOR EACH ROW
BEGIN
  declare diff BIGINT default 0;
  set diff = COALESCE(NEW.DiskSpace,0) - COALESCE(OLD.DiskSpace,0);

  IF ( diff ) THEN
    UPDATE Events_Hour SET DiskSpace=NEW.DiskSpace WHERE EventId=NEW.Id;
    UPDATE Events_Day SET DiskSpace=NEW.DiskSpace WHERE EventId=NEW.Id;
    UPDATE Events_Week SET DiskSpace=NEW.DiskSpace WHERE EventId=NEW.Id;
    UPDATE Events_Month SET DiskSpace=NEW.DiskSpace WHERE EventId=NEW.Id;
    UPDATE Event_Summaries
      SET
        TotalEventDiskSpace = GREATEST(COALESCE(TotalEventDiskSpace,0) - COALESCE(OLD.DiskSpace,0) + COALESCE(NEW.DiskSpace,0),0)
      WHERE Event_Summaries.MonitorId=OLD.MonitorId;
  END IF;

  IF ( NEW.Archived != OLD.Archived ) THEN
    IF ( NEW.Archived ) THEN
      INSERT INTO Events_Archived (EventId,MonitorId,DiskSpace) VALUES (NEW.Id,NEW.MonitorId,NEW.DiskSpace);
      INSERT INTO Event_Summaries (MonitorId,ArchivedEvents,ArchivedEventDiskSpace) VALUES (NEW.MonitorId,1,NEW.DiskSpace) ON DUPLICATE KEY
        UPDATE ArchivedEvents = COALESCE(ArchivedEvents,0)+1, ArchivedEventDiskSpace = COALESCE(ArchivedEventDiskSpace,0) + COALESCE(NEW.DiskSpace,0);
    ELSEIF ( OLD.Archived ) THEN
      DELETE FROM Events_Archived WHERE EventId=OLD.Id;
      UPDATE Event_Summaries
        SET
          ArchivedEvents = GREATEST(COALESCE(ArchivedEvents,0)-1,0),
          ArchivedEventDiskSpace = GREATEST(COALESCE(ArchivedEventDiskSpace,0) - COALESCE(OLD.DiskSpace,0),0)
        WHERE Event_Summaries.MonitorId=OLD.MonitorId;
    ELSE
      IF ( OLD.DiskSpace != NEW.DiskSpace ) THEN
        UPDATE Events_Archived SET DiskSpace=NEW.DiskSpace WHERE EventId=NEW.Id;
        UPDATE Event_Summaries SET
          ArchivedEventDiskSpace = GREATEST(COALESCE(ArchivedEventDiskSpace,0) - COALESCE(OLD.DiskSpace,0) + COALESCE(NEW.DiskSpace,0),0)
          WHERE Event_Summaries.MonitorId=OLD.MonitorId;
      END IF;
    END IF;
  ELSEIF ( NEW.Archived AND diff ) THEN
    UPDATE Events_Archived SET DiskSpace=NEW.DiskSpace WHERE EventId=NEW.Id;
  END IF;

END;

//

DROP TRIGGER IF EXISTS event_insert_trigger//

/* The assumption is that when an Event is inserted, it has no size yet, so don't bother updating the DiskSpace, just the count.
 * The DiskSpace will get update in the Event Update Trigger
 */
/*
CREATE TRIGGER event_insert_trigger AFTER INSERT ON Events
FOR EACH ROW
  BEGIN

  INSERT INTO Events_Hour (EventId,MonitorId,StartDateTime,DiskSpace) VALUES (NEW.Id,NEW.MonitorId,NEW.StartDateTime,0);
  INSERT INTO Events_Day (EventId,MonitorId,StartDateTime,DiskSpace) VALUES (NEW.Id,NEW.MonitorId,NEW.StartDateTime,0);
  INSERT INTO Events_Week (EventId,MonitorId,StartDateTime,DiskSpace) VALUES (NEW.Id,NEW.MonitorId,NEW.StartDateTime,0);
  INSERT INTO Events_Month (EventId,MonitorId,StartDateTime,DiskSpace) VALUES (NEW.Id,NEW.MonitorId,NEW.StartDateTime,0);
  INSERT INTO Event_Summaries (MonitorId,HourEvents,DayEvents,WeekEvents,MonthEvents,TotalEvents) VALUES (NEW.MonitorId,1,1,1,1,1) ON DUPLICATE KEY
  UPDATE 
  HourEvents = COALESCE(HourEvents,0)+1,
  DayEvents = COALESCE(DayEvents,0)+1,
  WeekEvents = COALESCE(WeekEvents,0)+1,
  MonthEvents = COALESCE(MonthEvents,0)+1,
  TotalEvents = COALESCE(TotalEvents,0)+1;
END;
//
*/

DROP TRIGGER IF EXISTS event_delete_trigger//

CREATE TRIGGER event_delete_trigger BEFORE DELETE ON Events
FOR EACH ROW
BEGIN
  DELETE FROM Events_Hour WHERE EventId=OLD.Id;
  DELETE FROM Events_Day WHERE EventId=OLD.Id;
  DELETE FROM Events_Week WHERE EventId=OLD.Id;
  DELETE FROM Events_Month WHERE EventId=OLD.Id;
  IF ( OLD.Archived ) THEN
    DELETE FROM Events_Archived WHERE EventId=OLD.Id;
    UPDATE Event_Summaries SET
      ArchivedEvents = GREATEST(COALESCE(ArchivedEvents,1) - 1,0),
      ArchivedEventDiskSpace = GREATEST(COALESCE(ArchivedEventDiskSpace,0) - COALESCE(OLD.DiskSpace,0),0),
      TotalEvents = GREATEST(COALESCE(TotalEvents,1) - 1,0),
      TotalEventDiskSpace = GREATEST(COALESCE(TotalEventDiskSpace,0) - COALESCE(OLD.DiskSpace,0),0)
      WHERE Event_Summaries.MonitorId=OLD.MonitorId;
  ELSE
    UPDATE Event_Summaries SET
    TotalEvents = GREATEST(COALESCE(TotalEvents,1)-1,0),
    TotalEventDiskSpace=GREATEST(COALESCE(TotalEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0)
    WHERE Event_Summaries.MonitorId=OLD.MonitorId;
  END IF;
END;

//

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
