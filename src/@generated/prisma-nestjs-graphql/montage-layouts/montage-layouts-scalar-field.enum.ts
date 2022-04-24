import { registerEnumType } from '@nestjs/graphql';

export enum MontageLayoutsScalarFieldEnum {
    Id = "Id",
    Name = "Name",
    Positions = "Positions"
}


registerEnumType(MontageLayoutsScalarFieldEnum, { name: 'MontageLayoutsScalarFieldEnum', description: undefined })
