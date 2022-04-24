import { registerEnumType } from '@nestjs/graphql';

export enum Users_Groups {
    None = "None",
    View = "View",
    Edit = "Edit"
}


registerEnumType(Users_Groups, { name: 'Users_Groups', description: undefined })
