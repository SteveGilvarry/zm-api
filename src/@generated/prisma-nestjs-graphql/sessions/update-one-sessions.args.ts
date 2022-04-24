import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SessionsUpdateInput } from './sessions-update.input';
import { SessionsWhereUniqueInput } from './sessions-where-unique.input';

@ArgsType()
export class UpdateOneSessionsArgs {

    @Field(() => SessionsUpdateInput, {nullable:false})
    data!: SessionsUpdateInput;

    @Field(() => SessionsWhereUniqueInput, {nullable:false})
    where!: SessionsWhereUniqueInput;
}
