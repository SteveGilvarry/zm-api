import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { TriggersX10WhereUniqueInput } from './triggers-x-10-where-unique.input';

@ArgsType()
export class FindUniqueTriggersX10Args {

    @Field(() => TriggersX10WhereUniqueInput, {nullable:false})
    where!: TriggersX10WhereUniqueInput;
}
