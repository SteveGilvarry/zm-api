import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsWhereUniqueInput } from './control-presets-where-unique.input';

@ArgsType()
export class FindUniqueControlPresetsArgs {

    @Field(() => ControlPresetsWhereUniqueInput, {nullable:false})
    where!: ControlPresetsWhereUniqueInput;
}
