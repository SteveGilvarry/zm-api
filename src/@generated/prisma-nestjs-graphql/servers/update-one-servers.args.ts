import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersUpdateInput } from './servers-update.input';
import { ServersWhereUniqueInput } from './servers-where-unique.input';

@ArgsType()
export class UpdateOneServersArgs {

    @Field(() => ServersUpdateInput, {nullable:false})
    data!: ServersUpdateInput;

    @Field(() => ServersWhereUniqueInput, {nullable:false})
    where!: ServersWhereUniqueInput;
}
