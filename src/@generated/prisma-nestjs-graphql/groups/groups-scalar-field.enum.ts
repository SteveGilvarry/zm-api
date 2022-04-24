import { registerEnumType } from '@nestjs/graphql';

export enum GroupsScalarFieldEnum {
    Id = "Id",
    Name = "Name",
    ParentId = "ParentId"
}


registerEnumType(GroupsScalarFieldEnum, { name: 'GroupsScalarFieldEnum', description: undefined })
