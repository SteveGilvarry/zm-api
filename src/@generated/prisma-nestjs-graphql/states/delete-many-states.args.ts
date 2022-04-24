import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesWhereInput } from './states-where.input';

@ArgsType()
export class DeleteManyStatesArgs {

    @Field(() => StatesWhereInput, {nullable:true})
    where?: StatesWhereInput;
}
