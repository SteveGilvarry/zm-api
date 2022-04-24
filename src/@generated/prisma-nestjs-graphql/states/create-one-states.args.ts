import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesCreateInput } from './states-create.input';

@ArgsType()
export class CreateOneStatesArgs {

    @Field(() => StatesCreateInput, {nullable:false})
    data!: StatesCreateInput;
}
