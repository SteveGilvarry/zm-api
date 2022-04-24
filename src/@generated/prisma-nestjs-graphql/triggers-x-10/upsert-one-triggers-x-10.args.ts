import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { TriggersX10WhereUniqueInput } from './triggers-x-10-where-unique.input';
import { TriggersX10CreateInput } from './triggers-x-10-create.input';
import { TriggersX10UpdateInput } from './triggers-x-10-update.input';

@ArgsType()
export class UpsertOneTriggersX10Args {

    @Field(() => TriggersX10WhereUniqueInput, {nullable:false})
    where!: TriggersX10WhereUniqueInput;

    @Field(() => TriggersX10CreateInput, {nullable:false})
    create!: TriggersX10CreateInput;

    @Field(() => TriggersX10UpdateInput, {nullable:false})
    update!: TriggersX10UpdateInput;
}
