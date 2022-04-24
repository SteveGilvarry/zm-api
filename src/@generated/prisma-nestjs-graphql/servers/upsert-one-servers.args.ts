import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersWhereUniqueInput } from './servers-where-unique.input';
import { ServersCreateInput } from './servers-create.input';
import { ServersUpdateInput } from './servers-update.input';

@ArgsType()
export class UpsertOneServersArgs {

    @Field(() => ServersWhereUniqueInput, {nullable:false})
    where!: ServersWhereUniqueInput;

    @Field(() => ServersCreateInput, {nullable:false})
    create!: ServersCreateInput;

    @Field(() => ServersUpdateInput, {nullable:false})
    update!: ServersUpdateInput;
}
