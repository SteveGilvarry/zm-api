import { registerEnumType } from '@nestjs/graphql';

export enum ConfigScalarFieldEnum {
    Id = "Id",
    Name = "Name",
    Value = "Value",
    Type = "Type",
    DefaultValue = "DefaultValue",
    Hint = "Hint",
    Pattern = "Pattern",
    Format = "Format",
    Prompt = "Prompt",
    Help = "Help",
    Category = "Category",
    Readonly = "Readonly",
    Requires = "Requires"
}


registerEnumType(ConfigScalarFieldEnum, { name: 'ConfigScalarFieldEnum', description: undefined })
