import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { TriggersX10UpdateInput } from './triggers-x-10-update.input';
import { TriggersX10WhereUniqueInput } from './triggers-x-10-where-unique.input';

@ArgsType()
export class UpdateOneTriggersX10Args {

    @Field(() => TriggersX10UpdateInput, {nullable:false})
    data!: TriggersX10UpdateInput;

    @Field(() => TriggersX10WhereUniqueInput, {nullable:false})
    where!: TriggersX10WhereUniqueInput;
}
