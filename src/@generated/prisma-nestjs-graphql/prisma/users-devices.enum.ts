import { registerEnumType } from '@nestjs/graphql';

export enum Users_Devices {
    None = "None",
    View = "View",
    Edit = "Edit"
}


registerEnumType(Users_Devices, { name: 'Users_Devices', description: undefined })
