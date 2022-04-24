import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsCreateInput } from './controls-create.input';

@ArgsType()
export class CreateOneControlsArgs {

    @Field(() => ControlsCreateInput, {nullable:false})
    data!: ControlsCreateInput;
}
