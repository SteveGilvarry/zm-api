import { registerEnumType } from '@nestjs/graphql';

export enum Storage_Type {
    local = "local",
    s3fs = "s3fs"
}


registerEnumType(Storage_Type, { name: 'Storage_Type', description: undefined })
