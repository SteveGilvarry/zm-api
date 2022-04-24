import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesUpdateManyMutationInput } from './states-update-many-mutation.input';
import { StatesWhereInput } from './states-where.input';

@ArgsType()
export class UpdateManyStatesArgs {

    @Field(() => StatesUpdateManyMutationInput, {nullable:false})
    data!: StatesUpdateManyMutationInput;

    @Field(() => StatesWhereInput, {nullable:true})
    where?: StatesWhereInput;
}
