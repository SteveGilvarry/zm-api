import { registerEnumType } from '@nestjs/graphql';

export enum Storage_Scheme {
    Deep = "Deep",
    Medium = "Medium",
    Shallow = "Shallow"
}


registerEnumType(Storage_Scheme, { name: 'Storage_Scheme', description: undefined })
