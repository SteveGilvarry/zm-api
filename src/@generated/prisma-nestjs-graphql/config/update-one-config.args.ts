import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigUpdateInput } from './config-update.input';
import { Type } from 'class-transformer';
import { ConfigWhereUniqueInput } from './config-where-unique.input';

@ArgsType()
export class UpdateOneConfigArgs {

    @Field(() => ConfigUpdateInput, {nullable:false})
    @Type(() => ConfigUpdateInput)
    data!: ConfigUpdateInput;

    @Field(() => ConfigWhereUniqueInput, {nullable:false})
    @Type(() => ConfigWhereUniqueInput)
    where!: ConfigWhereUniqueInput;
}
