import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesUpdateInput } from './states-update.input';
import { Type } from 'class-transformer';
import { StatesWhereUniqueInput } from './states-where-unique.input';

@ArgsType()
export class UpdateOneStatesArgs {

    @Field(() => StatesUpdateInput, {nullable:false})
    @Type(() => StatesUpdateInput)
    data!: StatesUpdateInput;

    @Field(() => StatesWhereUniqueInput, {nullable:false})
    @Type(() => StatesWhereUniqueInput)
    where!: StatesWhereUniqueInput;
}
