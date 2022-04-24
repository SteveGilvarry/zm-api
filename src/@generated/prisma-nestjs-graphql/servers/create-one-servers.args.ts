import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersCreateInput } from './servers-create.input';

@ArgsType()
export class CreateOneServersArgs {

    @Field(() => ServersCreateInput, {nullable:false})
    data!: ServersCreateInput;
}
