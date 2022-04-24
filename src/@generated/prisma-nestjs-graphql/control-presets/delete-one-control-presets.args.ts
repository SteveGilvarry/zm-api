import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsWhereUniqueInput } from './control-presets-where-unique.input';

@ArgsType()
export class DeleteOneControlPresetsArgs {

    @Field(() => ControlPresetsWhereUniqueInput, {nullable:false})
    where!: ControlPresetsWhereUniqueInput;
}
