import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesWhereUniqueInput } from './states-where-unique.input';
import { StatesCreateInput } from './states-create.input';
import { StatesUpdateInput } from './states-update.input';

@ArgsType()
export class UpsertOneStatesArgs {

    @Field(() => StatesWhereUniqueInput, {nullable:false})
    where!: StatesWhereUniqueInput;

    @Field(() => StatesCreateInput, {nullable:false})
    create!: StatesCreateInput;

    @Field(() => StatesUpdateInput, {nullable:false})
    update!: StatesUpdateInput;
}
