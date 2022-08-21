import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesWhereUniqueInput } from './states-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneStatesArgs {

    @Field(() => StatesWhereUniqueInput, {nullable:false})
    @Type(() => StatesWhereUniqueInput)
    where!: StatesWhereUniqueInput;
}
