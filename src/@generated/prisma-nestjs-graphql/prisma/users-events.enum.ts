import { registerEnumType } from '@nestjs/graphql';

export enum Users_Events {
    None = "None",
    View = "View",
    Edit = "Edit"
}


registerEnumType(Users_Events, { name: 'Users_Events', description: undefined })
