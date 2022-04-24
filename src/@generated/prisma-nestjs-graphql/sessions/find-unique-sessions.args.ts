import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SessionsWhereUniqueInput } from './sessions-where-unique.input';

@ArgsType()
export class FindUniqueSessionsArgs {

    @Field(() => SessionsWhereUniqueInput, {nullable:false})
    where!: SessionsWhereUniqueInput;
}
