import { registerEnumType } from '@nestjs/graphql';

export enum StorageScalarFieldEnum {
    Id = "Id",
    Path = "Path",
    Name = "Name",
    Type = "Type",
    Url = "Url",
    DiskSpace = "DiskSpace",
    Scheme = "Scheme",
    ServerId = "ServerId",
    DoDelete = "DoDelete",
    Enabled = "Enabled"
}


registerEnumType(StorageScalarFieldEnum, { name: 'StorageScalarFieldEnum', description: undefined })
