import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsWhereUniqueInput } from './controls-where-unique.input';

@ArgsType()
export class FindUniqueControlsArgs {

    @Field(() => ControlsWhereUniqueInput, {nullable:false})
    where!: ControlsWhereUniqueInput;
}
