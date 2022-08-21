import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigWhereUniqueInput } from './config-where-unique.input';
import { Type } from 'class-transformer';
import { ConfigCreateInput } from './config-create.input';
import { ConfigUpdateInput } from './config-update.input';

@ArgsType()
export class UpsertOneConfigArgs {

    @Field(() => ConfigWhereUniqueInput, {nullable:false})
    @Type(() => ConfigWhereUniqueInput)
    where!: ConfigWhereUniqueInput;

    @Field(() => ConfigCreateInput, {nullable:false})
    @Type(() => ConfigCreateInput)
    create!: ConfigCreateInput;

    @Field(() => ConfigUpdateInput, {nullable:false})
    @Type(() => ConfigUpdateInput)
    update!: ConfigUpdateInput;
}
