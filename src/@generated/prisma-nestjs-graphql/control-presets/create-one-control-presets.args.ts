import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsCreateInput } from './control-presets-create.input';

@ArgsType()
export class CreateOneControlPresetsArgs {

    @Field(() => ControlPresetsCreateInput, {nullable:false})
    data!: ControlPresetsCreateInput;
}
