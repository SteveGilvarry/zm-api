import { registerEnumType } from '@nestjs/graphql';

export enum Users_System {
    None = "None",
    View = "View",
    Edit = "Edit"
}


registerEnumType(Users_System, { name: 'Users_System', description: undefined })
