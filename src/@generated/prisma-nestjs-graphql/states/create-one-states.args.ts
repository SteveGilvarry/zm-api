import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StatesCreateInput } from './states-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneStatesArgs {

    @Field(() => StatesCreateInput, {nullable:false})
    @Type(() => StatesCreateInput)
    data!: StatesCreateInput;
}
