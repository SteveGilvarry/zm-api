import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsCreateInput } from './control-presets-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneControlPresetsArgs {

    @Field(() => ControlPresetsCreateInput, {nullable:false})
    @Type(() => ControlPresetsCreateInput)
    data!: ControlPresetsCreateInput;
}
