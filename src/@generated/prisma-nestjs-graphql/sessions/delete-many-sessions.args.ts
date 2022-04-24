import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SessionsWhereInput } from './sessions-where.input';

@ArgsType()
export class DeleteManySessionsArgs {

    @Field(() => SessionsWhereInput, {nullable:true})
    where?: SessionsWhereInput;
}
