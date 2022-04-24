import { registerEnumType } from '@nestjs/graphql';

export enum DevicesScalarFieldEnum {
    Id = "Id",
    Name = "Name",
    Type = "Type",
    KeyString = "KeyString"
}


registerEnumType(DevicesScalarFieldEnum, { name: 'DevicesScalarFieldEnum', description: undefined })
