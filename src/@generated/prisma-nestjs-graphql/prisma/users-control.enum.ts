import { registerEnumType } from '@nestjs/graphql';

export enum Users_Control {
    None = "None",
    View = "View",
    Edit = "Edit"
}


registerEnumType(Users_Control, { name: 'Users_Control', description: undefined })
