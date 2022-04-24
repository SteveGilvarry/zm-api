import { registerEnumType } from '@nestjs/graphql';

export enum Events_Scheme {
    Deep = "Deep",
    Medium = "Medium",
    Shallow = "Shallow"
}


registerEnumType(Events_Scheme, { name: 'Events_Scheme', description: undefined })
