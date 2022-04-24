import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesWhereUniqueInput } from './states-where-unique.input';

@ArgsType()
export class FindUniqueStatesArgs {

    @Field(() => StatesWhereUniqueInput, {nullable:false})
    where!: StatesWhereUniqueInput;
}
