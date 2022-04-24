import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersWhereUniqueInput } from './servers-where-unique.input';

@ArgsType()
export class DeleteOneServersArgs {

    @Field(() => ServersWhereUniqueInput, {nullable:false})
    where!: ServersWhereUniqueInput;
}
