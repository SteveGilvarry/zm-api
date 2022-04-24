import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersWhereInput } from './servers-where.input';

@ArgsType()
export class DeleteManyServersArgs {

    @Field(() => ServersWhereInput, {nullable:true})
    where?: ServersWhereInput;
}
