import { registerEnumType } from '@nestjs/graphql';

export enum ModelsScalarFieldEnum {
    Id = "Id",
    Name = "Name",
    ManufacturerId = "ManufacturerId"
}


registerEnumType(ModelsScalarFieldEnum, { name: 'ModelsScalarFieldEnum', description: undefined })
