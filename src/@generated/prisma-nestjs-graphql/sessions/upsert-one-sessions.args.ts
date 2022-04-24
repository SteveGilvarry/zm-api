import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SessionsWhereUniqueInput } from './sessions-where-unique.input';
import { SessionsCreateInput } from './sessions-create.input';
import { SessionsUpdateInput } from './sessions-update.input';

@ArgsType()
export class UpsertOneSessionsArgs {

    @Field(() => SessionsWhereUniqueInput, {nullable:false})
    where!: SessionsWhereUniqueInput;

    @Field(() => SessionsCreateInput, {nullable:false})
    create!: SessionsCreateInput;

    @Field(() => SessionsUpdateInput, {nullable:false})
    update!: SessionsUpdateInput;
}
