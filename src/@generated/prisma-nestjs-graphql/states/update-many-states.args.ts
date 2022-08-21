import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesUpdateManyMutationInput } from './states-update-many-mutation.input';
import { Type } from 'class-transformer';
import { StatesWhereInput } from './states-where.input';

@ArgsType()
export class UpdateManyStatesArgs {

    @Field(() => StatesUpdateManyMutationInput, {nullable:false})
    @Type(() => StatesUpdateManyMutationInput)
    data!: StatesUpdateManyMutationInput;

    @Field(() => StatesWhereInput, {nullable:true})
    @Type(() => StatesWhereInput)
    where?: StatesWhereInput;
}
