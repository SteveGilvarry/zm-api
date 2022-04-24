import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesUpdateInput } from './states-update.input';
import { StatesWhereUniqueInput } from './states-where-unique.input';

@ArgsType()
export class UpdateOneStatesArgs {

    @Field(() => StatesUpdateInput, {nullable:false})
    data!: StatesUpdateInput;

    @Field(() => StatesWhereUniqueInput, {nullable:false})
    where!: StatesWhereUniqueInput;
}
