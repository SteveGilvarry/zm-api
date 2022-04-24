import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SessionsCreateInput } from './sessions-create.input';

@ArgsType()
export class CreateOneSessionsArgs {

    @Field(() => SessionsCreateInput, {nullable:false})
    data!: SessionsCreateInput;
}
