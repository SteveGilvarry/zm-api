import { registerEnumType } from '@nestjs/graphql';

export enum FiltersScalarFieldEnum {
    Id = "Id",
    Name = "Name",
    UserId = "UserId",
    Query_json = "Query_json",
    AutoArchive = "AutoArchive",
    AutoUnarchive = "AutoUnarchive",
    AutoVideo = "AutoVideo",
    AutoUpload = "AutoUpload",
    AutoEmail = "AutoEmail",
    EmailTo = "EmailTo",
    EmailSubject = "EmailSubject",
    EmailBody = "EmailBody",
    AutoMessage = "AutoMessage",
    AutoExecute = "AutoExecute",
    AutoExecuteCmd = "AutoExecuteCmd",
    AutoDelete = "AutoDelete",
    AutoMove = "AutoMove",
    AutoCopy = "AutoCopy",
    AutoCopyTo = "AutoCopyTo",
    AutoMoveTo = "AutoMoveTo",
    UpdateDiskSpace = "UpdateDiskSpace",
    Background = "Background",
    Concurrent = "Concurrent",
    LockRows = "LockRows"
}


registerEnumType(FiltersScalarFieldEnum, { name: 'FiltersScalarFieldEnum', description: undefined })
