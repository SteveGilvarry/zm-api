import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesWhereInput } from './states-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyStatesArgs {

    @Field(() => StatesWhereInput, {nullable:true})
    @Type(() => StatesWhereInput)
    where?: StatesWhereInput;
}
