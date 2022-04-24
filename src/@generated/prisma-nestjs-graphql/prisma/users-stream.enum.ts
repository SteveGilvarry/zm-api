import { registerEnumType } from '@nestjs/graphql';

export enum Users_Stream {
    None = "None",
    View = "View"
}


registerEnumType(Users_Stream, { name: 'Users_Stream', description: undefined })
