import { registerEnumType } from '@nestjs/graphql';

export enum ManufacturersScalarFieldEnum {
    Id = "Id",
    Name = "Name"
}


registerEnumType(ManufacturersScalarFieldEnum, { name: 'ManufacturersScalarFieldEnum', description: undefined })
