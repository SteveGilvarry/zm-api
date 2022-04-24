import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigWhereUniqueInput } from './config-where-unique.input';
import { ConfigCreateInput } from './config-create.input';
import { ConfigUpdateInput } from './config-update.input';

@ArgsType()
export class UpsertOneConfigArgs {

    @Field(() => ConfigWhereUniqueInput, {nullable:false})
    where!: ConfigWhereUniqueInput;

    @Field(() => ConfigCreateInput, {nullable:false})
    create!: ConfigCreateInput;

    @Field(() => ConfigUpdateInput, {nullable:false})
    update!: ConfigUpdateInput;
}
