import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesWhereUniqueInput } from './states-where-unique.input';
import { Type } from 'class-transformer';
import { StatesCreateInput } from './states-create.input';
import { StatesUpdateInput } from './states-update.input';

@ArgsType()
export class UpsertOneStatesArgs {

    @Field(() => StatesWhereUniqueInput, {nullable:false})
    @Type(() => StatesWhereUniqueInput)
    where!: StatesWhereUniqueInput;

    @Field(() => StatesCreateInput, {nullable:false})
    @Type(() => StatesCreateInput)
    create!: StatesCreateInput;

    @Field(() => StatesUpdateInput, {nullable:false})
    @Type(() => StatesUpdateInput)
    update!: StatesUpdateInput;
}
