import { registerEnumType } from '@nestjs/graphql';

export enum SessionsScalarFieldEnum {
    id = "id",
    access = "access",
    data = "data"
}


registerEnumType(SessionsScalarFieldEnum, { name: 'SessionsScalarFieldEnum', description: undefined })
