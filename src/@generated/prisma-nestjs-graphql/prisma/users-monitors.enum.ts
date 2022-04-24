import { registerEnumType } from '@nestjs/graphql';

export enum Users_Monitors {
    None = "None",
    View = "View",
    Edit = "Edit"
}


registerEnumType(Users_Monitors, { name: 'Users_Monitors', description: undefined })
