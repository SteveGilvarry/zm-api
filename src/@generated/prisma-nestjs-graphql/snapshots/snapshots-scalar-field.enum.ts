import { registerEnumType } from '@nestjs/graphql';

export enum SnapshotsScalarFieldEnum {
    Id = "Id",
    Name = "Name",
    Description = "Description",
    CreatedBy = "CreatedBy",
    CreatedOn = "CreatedOn"
}


registerEnumType(SnapshotsScalarFieldEnum, { name: 'SnapshotsScalarFieldEnum', description: undefined })
