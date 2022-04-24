import { registerEnumType } from '@nestjs/graphql';

export enum Users_Snapshots {
    None = "None",
    View = "View",
    Edit = "Edit"
}


registerEnumType(Users_Snapshots, { name: 'Users_Snapshots', description: undefined })
