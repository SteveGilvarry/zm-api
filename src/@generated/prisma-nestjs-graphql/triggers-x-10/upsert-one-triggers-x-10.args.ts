import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { TriggersX10WhereUniqueInput } from './triggers-x-10-where-unique.input';
import { Type } from 'class-transformer';
import { TriggersX10CreateInput } from './triggers-x-10-create.input';
import { TriggersX10UpdateInput } from './triggers-x-10-update.input';

@ArgsType()
export class UpsertOneTriggersX10Args {

    @Field(() => TriggersX10WhereUniqueInput, {nullable:false})
    @Type(() => TriggersX10WhereUniqueInput)
    where!: TriggersX10WhereUniqueInput;

    @Field(() => TriggersX10CreateInput, {nullable:false})
    @Type(() => TriggersX10CreateInput)
    create!: TriggersX10CreateInput;

    @Field(() => TriggersX10UpdateInput, {nullable:false})
    @Type(() => TriggersX10UpdateInput)
    update!: TriggersX10UpdateInput;
}
