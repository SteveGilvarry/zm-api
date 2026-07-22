//! The 13 MySQL triggers that maintain Event_Summaries and the
//! Events_Hour/Day/Week/Month rollup tables, plus Zones counts,
//! extracted from db/triggers.sql. MySQL dialect only - the Postgres
//! port (functions + triggers) lands with the Postgres schema work;
//! until then Postgres installs run without summary triggers.

pub(super) fn mysql_triggers() -> Vec<&'static str> {
    vec![
        r#"CREATE TRIGGER Events_Hour_delete_trigger BEFORE DELETE ON Events_Hour
FOR EACH ROW BEGIN
  UPDATE Event_Summaries SET
  HourEvents = GREATEST(COALESCE(HourEvents,1)-1,0),
  HourEventDiskSpace=GREATEST(COALESCE(HourEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0)
  WHERE Event_Summaries.MonitorId=OLD.MonitorId;
END
"#,
        r#"CREATE TRIGGER Events_Hour_update_trigger AFTER UPDATE ON Events_Hour
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
  END
"#,
        r#"CREATE TRIGGER Events_Day_delete_trigger BEFORE DELETE ON Events_Day
FOR EACH ROW BEGIN
  UPDATE Event_Summaries SET
  DayEvents = GREATEST(COALESCE(DayEvents,1)-1,0),
  DayEventDiskSpace=GREATEST(COALESCE(DayEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0)
  WHERE Event_Summaries.MonitorId=OLD.MonitorId;
END
"#,
        r#"CREATE TRIGGER Events_Day_update_trigger AFTER UPDATE ON Events_Day
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
  END
"#,
        r#"CREATE TRIGGER Events_Week_delete_trigger BEFORE DELETE ON Events_Week
FOR EACH ROW BEGIN
  UPDATE Event_Summaries SET
  WeekEvents = GREATEST(COALESCE(WeekEvents,1)-1,0),
  WeekEventDiskSpace=GREATEST(COALESCE(WeekEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0)
  WHERE Event_Summaries.MonitorId=OLD.MonitorId;
END
"#,
        r#"CREATE TRIGGER Events_Week_update_trigger AFTER UPDATE ON Events_Week
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
  END
"#,
        r#"CREATE TRIGGER Events_Month_delete_trigger BEFORE DELETE ON Events_Month
FOR EACH ROW BEGIN
  UPDATE Event_Summaries SET
  MonthEvents = GREATEST(COALESCE(MonthEvents,1)-1,0),
  MonthEventDiskSpace=GREATEST(COALESCE(MonthEventDiskSpace,0)-COALESCE(OLD.DiskSpace,0),0)
  WHERE Event_Summaries.MonitorId=OLD.MonitorId;
END
"#,
        r#"CREATE TRIGGER Events_Month_update_trigger AFTER UPDATE ON Events_Month
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
  END
"#,
        r#"CREATE TRIGGER event_update_trigger AFTER UPDATE ON Events
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

END
"#,
        r#"CREATE TRIGGER event_delete_trigger BEFORE DELETE ON Events
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
END
"#,
        r#"CREATE TRIGGER Zone_Insert_Trigger AFTER INSERT ON Zones
FOR EACH ROW
  BEGIN
    UPDATE Monitors SET ZoneCount=(SELECT COUNT(*) FROM Zones WHERE MonitorId=NEW.MonitorId) WHERE Monitors.Id=NEW.MonitorID;
  END
"#,
        r#"CREATE TRIGGER Zone_Delete_Trigger AFTER DELETE ON Zones
FOR EACH ROW
  BEGIN
    UPDATE Monitors SET ZoneCount=(SELECT COUNT(*) FROM Zones WHERE MonitorId=OLD.MonitorId) WHERE Monitors.Id=OLD.MonitorID;
  END
"#,
    ]
}

/// Tables with an auto-increment id whose seeds insert explicit ids;
/// Postgres sequences must be advanced past them after seeding.
pub(super) fn autoinc_tables() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Controls", "Id"),
        ("Devices", "Id"),
        ("EncoderTemplates", "Id"),
        ("Events", "Id"),
        ("Event_Data", "Id"),
        ("Filters", "Id"),
        ("Frames", "Id"),
        ("Groups", "Id"),
        ("Groups_Monitors", "Id"),
        ("Groups_Permissions", "Id"),
        ("Monitors_Permissions", "Id"),
        ("Role_Groups_Permissions", "Id"),
        ("Role_Monitors_Permissions", "Id"),
        ("Logs", "Id"),
        ("Manufacturers", "Id"),
        ("Models", "Id"),
        ("MonitorPresets", "Id"),
        ("Monitors", "Id"),
        ("States", "Id"),
        ("Servers", "Id"),
        ("Server_Stats", "Id"),
        ("Stats", "Id"),
        ("User_Roles", "Id"),
        ("Users", "Id"),
        ("User_Preferences", "Id"),
        ("ZonePresets", "Id"),
        ("Zones", "Id"),
        ("Storage", "Id"),
        ("Maps", "Id"),
        ("MontageLayouts", "Id"),
        ("Snapshots", "Id"),
        ("Snapshots_Events", "Id"),
        ("Reports", "Id"),
        ("Tags", "Id"),
        ("Notifications", "Id"),
        ("Menu_Items", "Id"),
        ("Object_Types", "Id"),
        ("AI_Datasets", "Id"),
        ("AI_Models", "Id"),
        ("AI_Object_Classes", "Id"),
        ("AI_Detection_Settings", "Id"),
        ("AI_Detections", "Id"),
    ]
}
