import { registerEnumType } from '@nestjs/graphql';

export enum StatesScalarFieldEnum {
    Id = "Id",
    Name = "Name",
    Definition = "Definition",
    IsActive = "IsActive"
}


registerEnumType(StatesScalarFieldEnum, { name: 'StatesScalarFieldEnum', description: undefined })
